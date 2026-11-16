//! RUN-3's operating-system sandbox for the test or simulation child that `--tide-run-test` and
//! `--replay` start: Landlock and seccomp, both allowlists, installed between fork and exec, so
//! they cover everything the binary runs, its loader and static constructors included.
//! `design/03-authority-concurrency.md` RUN-3 owns the policy. This module reports the level the
//! host gets (`full`, or `partial` naming what the kernel cannot enforce), and the start fails
//! when the child cannot install that level.
//!
//! Landlock lets the child open by path only its binary, the loader and libraries the binary
//! names, to read and execute, its context, to read, and its fresh run directory's `sim/`
//! (`RunDir`), where it may create only regular files and directories; its scopes keep signals and abstract Unix sockets within the child, and
//! it handles TCP and UDP ports with no port allowed. seccomp admits only the system calls the
//! runtime needs and fails every other with `EPERM` (`clone3` with `ENOSYS`, so the C library
//! falls back to `clone`, whose flags the filter reads).
use std::ffi::CStr;
use std::fs;
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) struct Sandbox {
    level: String,
    ruleset: Option<OwnedFd>,
    filter: Option<Vec<libc::sock_filter>>,
}

impl Sandbox {
    /// Probes the host and builds the policy for the run in `run`: its verified binary copy
    /// (`test`) and context (`context.json`) to read, and its `sim/` to write.
    pub(super) fn prepare(run: &RunDir) -> Result<Self, String> {
        let mut core = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
        // SAFETY: `getrlimit` writes only the struct it is given.
        if unsafe { libc::getrlimit(libc::RLIMIT_CORE, &mut core) } != 0 || core.rlim_max < 1 {
            return Err("the hard core-size limit is below one byte, and the harness cannot \
                        raise it to the one byte RUN-3 sets"
                .into());
        }
        // SAFETY: an empty range closes nothing; this only asks whether the kernel has the call.
        let cloexec = libc::CLOSE_RANGE_CLOEXEC as libc::c_uint;
        if unsafe { libc::syscall(libc::SYS_close_range, u32::MAX, u32::MAX, cloexec) } != 0 {
            return Err(format!(
                "the kernel lacks close_range with CLOSE_RANGE_CLOEXEC (Linux 5.11), so the \
                 harness cannot close the descriptors the child would inherit: {}",
                io::Error::last_os_error()
            ));
        }
        let abi = landlock_abi();
        let seccomp = seccomp_available();
        let level = level(&kernel_release(), abi, seccomp);
        let ruleset = match abi {
            Some(abi) => Some(
                ruleset(abi, run, &libraries(&run.path.join("test"))?)
                    .map_err(|e| format!("cannot build the Landlock ruleset: {e}"))?,
            ),
            None => None,
        };
        Ok(Self {
            level,
            ruleset,
            filter: seccomp.then(filter),
        })
    }

    /// `full (…)` or `partial, not enforced: … (…)`, with the kernel release and Landlock ABI.
    pub(super) fn level(&self) -> &str {
        &self.level
    }

    /// Installs the sandbox in `command`'s child between fork and exec; a step that fails fails
    /// the spawn with its `errno`.
    pub(super) fn apply(self, command: &mut Command) {
        let Self { ruleset, filter, .. } = self;
        // SAFETY: `install` makes only async-signal-safe system calls and allocates nothing.
        unsafe {
            let parent = std::process::id() as libc::pid_t;
            command.pre_exec(move || install(ruleset.as_ref(), filter.as_deref(), parent));
        }
    }
}

/// The level a host with this Landlock ABI and seccomp support gets (RUN-3).
fn level(kernel: &str, abi: Option<u32>, seccomp: bool) -> String {
    let mut missing = Vec::new();
    match abi {
        None => missing.push("Landlock"),
        Some(abi) if abi < 6 => {
            missing.push("Landlock scopes for signals and abstract Unix sockets (ABI 6)")
        }
        Some(_) => {}
    }
    if !seccomp {
        missing.push("seccomp");
    }
    let landlock = abi.map_or_else(|| "no Landlock".into(), |abi| format!("Landlock ABI {abi}"));
    if missing.is_empty() {
        format!("full (Linux {kernel}, {landlock})")
    } else {
        format!("partial, not enforced: {} (Linux {kernel}, {landlock})", missing.join(", "))
    }
}

fn check(result: libc::c_long) -> io::Result<()> {
    if result < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// Runs in the forked child: a new session with no controlling terminal, core dumps off (soft and
/// hard limits at one byte, which survive exec), no inherited descriptor past the standard three,
/// then Landlock and seccomp.
fn install(
    ruleset: Option<&OwnedFd>,
    filter: Option<&[libc::sock_filter]>,
    parent: libc::pid_t,
) -> io::Result<()> {
    // SAFETY: each call passes valid arguments and touches no memory but what it is given.
    unsafe {
        check(libc::setsid().into())?;
        // The new session leaves the caller's process group, so a terminal's interrupt or
        // `timeout` no longer reaches the child: it dies with the thread that spawned it, which
        // also waits on it (`supervise`). A parent that died before this line is caught below.
        check(libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0).into())?;
        if libc::getppid() != parent {
            libc::_exit(1);
        }
        let core = libc::rlimit { rlim_cur: 1, rlim_max: 1 };
        check(libc::setrlimit(libc::RLIMIT_CORE, &core).into())?;
        // `prepare` checked that the kernel has this; any failure fails the start.
        let cloexec = libc::CLOSE_RANGE_CLOEXEC as libc::c_uint;
        check(libc::syscall(libc::SYS_close_range, 3u32, u32::MAX, cloexec))?;
        check(libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0).into())?;
        if let Some(ruleset) = ruleset {
            check(libc::syscall(libc::SYS_landlock_restrict_self, ruleset.as_raw_fd(), 0u32))?;
        }
        if let Some(filter) = filter {
            let program = libc::sock_fprog {
                len: filter.len() as libc::c_ushort,
                filter: filter.as_ptr().cast_mut(),
            };
            let mode = libc::SECCOMP_SET_MODE_FILTER as libc::c_uint;
            check(libc::syscall(libc::SYS_seccomp, mode, 0u32, &program))?;
        }
    }
    Ok(())
}

fn kernel_release() -> String {
    // SAFETY: `uname` fills the struct; `release` is NUL-terminated on success.
    unsafe {
        let mut name: libc::utsname = std::mem::zeroed();
        if libc::uname(&mut name) != 0 {
            return "(unknown release)".into();
        }
        CStr::from_ptr(name.release.as_ptr()).to_string_lossy().into_owned()
    }
}

// Landlock's interface (include/uapi/linux/landlock.h).
const CREATE_RULESET_VERSION: libc::c_uint = 1;
const RULE_PATH_BENEATH: libc::c_uint = 1;
const FS_EXECUTE: u64 = 1 << 0;
const FS_WRITE_FILE: u64 = 1 << 1;
const FS_READ_FILE: u64 = 1 << 2;
const FS_READ_DIR: u64 = 1 << 3;
const FS_REMOVE_FILE: u64 = 1 << 5;
const FS_MAKE_DIR: u64 = 1 << 7;
const FS_MAKE_REG: u64 = 1 << 8;
const FS_REFER: u64 = 1 << 13;
const FS_TRUNCATE: u64 = 1 << 14;
const FS_IOCTL_DEV: u64 = 1 << 15;
const FS_RESOLVE_UNIX: u64 = 1 << 16;
const NET_TCP: u64 = 0b11;
const NET_UDP: u64 = 0b1100;
const SCOPES: u64 = 0b11;

#[repr(C)]
struct RulesetAttr {
    handled_access_fs: u64,
    handled_access_net: u64,
    scoped: u64,
}

#[repr(C, packed)]
struct PathBeneath {
    allowed_access: u64,
    parent_fd: i32,
}

fn landlock_abi() -> Option<u32> {
    // SAFETY: the version query reads no memory.
    let abi = unsafe {
        libc::syscall(
            libc::SYS_landlock_create_ruleset,
            std::ptr::null::<RulesetAttr>(),
            0usize,
            CREATE_RULESET_VERSION,
        )
    };
    u32::try_from(abi).ok().filter(|&abi| abi >= 1)
}

/// Every filesystem right the kernel's ABI knows, up to the ABI this driver knows (10), so an
/// unlisted operation is denied rather than left unhandled.
fn handled_fs(abi: u32) -> u64 {
    let mut rights = (1 << 13) - 1; // ABI 1: execute through make-symlink
    for (since, right) in [(2, FS_REFER), (3, FS_TRUNCATE), (5, FS_IOCTL_DEV), (9, FS_RESOLVE_UNIX)] {
        if abi >= since {
            rights |= right;
        }
    }
    rights
}

fn ruleset(abi: u32, run: &RunDir, libraries: &[PathBuf]) -> io::Result<OwnedFd> {
    let handled = handled_fs(abi);
    let attr = RulesetAttr {
        handled_access_fs: handled,
        handled_access_net: if abi >= 10 { NET_TCP | NET_UDP } else if abi >= 4 { NET_TCP } else { 0 },
        scoped: if abi >= 6 { SCOPES } else { 0 },
    };
    // SAFETY: `attr` is a valid ruleset attribute of the size passed.
    let fd = unsafe {
        libc::syscall(
            libc::SYS_landlock_create_ruleset,
            &attr,
            size_of::<RulesetAttr>(),
            0u32,
        )
    };
    check(fd)?;
    // SAFETY: the kernel returned a fresh descriptor, owned only here.
    let ruleset = unsafe { OwnedFd::from_raw_fd(fd as libc::c_int) };
    let write = FS_READ_FILE | FS_WRITE_FILE | FS_READ_DIR | FS_MAKE_REG | FS_MAKE_DIR
        | FS_REMOVE_FILE | FS_TRUNCATE;
    let mut rules = vec![
        (run.open("test", libc::O_PATH)?, FS_EXECUTE | FS_READ_FILE),
        (run.open("context.json", libc::O_PATH)?, FS_READ_FILE),
        (run.open("sim", libc::O_PATH | libc::O_DIRECTORY)?, write),
    ];
    for library in libraries {
        let file = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_PATH | libc::O_CLOEXEC)
            .open(library)?;
        rules.push((file.into(), FS_EXECUTE | FS_READ_FILE));
    }
    for (file, access) in rules {
        let rule = PathBeneath {
            allowed_access: access & handled,
            parent_fd: file.as_raw_fd(),
        };
        // SAFETY: `rule` is a valid path-beneath attribute for the open `file`.
        check(unsafe {
            libc::syscall(
                libc::SYS_landlock_add_rule,
                ruleset.as_raw_fd(),
                RULE_PATH_BENEATH,
                &rule,
                0u32,
            )
        })?;
    }
    Ok(ruleset)
}

/// A fresh private directory for one run, in the driver's state under `OUT/.tide-cache`
/// (RUN-2): the binary copy, the context, and `sim/`, the only directory the child may write.
/// Every directory is created and opened without following a link.
pub(super) struct RunDir {
    pub(super) path: PathBuf,
    fd: OwnedFd,
}

impl RunDir {
    pub(super) fn create(cache: &Path) -> Result<Self, String> {
        let error = |e: io::Error| format!("cannot create the run directory in {}: {e}", cache.display());
        let parent = cache.parent().ok_or("invalid cache directory")?;
        let name = cache.file_name().ok_or("invalid cache directory")?;
        let cache_fd = subdirectory(&open_directory(parent).map_err(error)?, name.as_encoded_bytes())
            .map_err(error)?;
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        for attempt in 0..16 {
            let name = format!("run-{}-{nonce}-{attempt}", std::process::id());
            // SAFETY: `name` is NUL-terminated by `c_name`; `cache_fd` is open.
            let made = unsafe { libc::mkdirat(cache_fd.as_raw_fd(), c_name(name.as_bytes()).as_ptr(), 0o700) };
            if made != 0 {
                let e = io::Error::last_os_error();
                if e.kind() == io::ErrorKind::AlreadyExists {
                    continue;
                }
                return Err(error(e));
            }
            let fd = openat(&cache_fd, name.as_bytes(), libc::O_RDONLY | libc::O_DIRECTORY).map_err(error)?;
            let run = Self { path: cache.join(&name), fd };
            subdirectory(&run.fd, b"sim").map_err(error)?;
            return Ok(run);
        }
        Err(error(io::ErrorKind::AlreadyExists.into()))
    }

    fn open(&self, name: &str, flags: libc::c_int) -> io::Result<OwnedFd> {
        openat(&self.fd, name.as_bytes(), flags)
    }

    /// The run's token: the harness-chosen prefix of every name this run may publish.
    pub(super) fn token(&self) -> String {
        self.path.file_name().unwrap_or_default().to_string_lossy().into_owned()
    }

    /// Moves each directory the child left in `sim/` whose name carries this run's token to
    /// `out/target/sim/`, never replacing an existing entry, so a child can publish only into its
    /// own run's namespace (RUN-2). `target` and `target/sim` are opened without following a
    /// link; a link or file there fails the publication and is left alone. The run directory is
    /// removed afterward.
    pub(super) fn publish(self, out: &Path) -> Result<(), String> {
        let error = |e: io::Error| format!("cannot publish run artifacts to {}: {e}", out.join("target/sim").display());
        let prefix = format!("{}-", self.token());
        let result = (|| {
            let target = subdirectory(&open_directory(out)?, b"target")?;
            let published = subdirectory(&target, b"sim")?;
            let sim = self.open("sim", libc::O_RDONLY | libc::O_DIRECTORY)?;
            for entry in fs::read_dir(self.path.join("sim"))? {
                let entry = entry?;
                let name = entry.file_name();
                if !entry.file_type()?.is_dir() || !name.as_encoded_bytes().starts_with(prefix.as_bytes()) {
                    continue;
                }
                let name = c_name(name.as_encoded_bytes());
                // SAFETY: valid descriptors and NUL-terminated names.
                check(unsafe {
                    libc::syscall(
                        libc::SYS_renameat2,
                        sim.as_raw_fd(),
                        name.as_ptr(),
                        published.as_raw_fd(),
                        name.as_ptr(),
                        libc::RENAME_NOREPLACE,
                    )
                })?;
            }
            Ok(())
        })()
        .map_err(error);
        self.remove();
        result
    }

    pub(super) fn remove(self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn c_name(name: &[u8]) -> std::ffi::CString {
    std::ffi::CString::new(name).unwrap_or_default()
}

fn openat(parent: &OwnedFd, name: &[u8], flags: libc::c_int) -> io::Result<OwnedFd> {
    let name = c_name(name);
    // SAFETY: a valid descriptor and a NUL-terminated name.
    let fd = unsafe {
        libc::openat(parent.as_raw_fd(), name.as_ptr(), flags | libc::O_NOFOLLOW | libc::O_CLOEXEC)
    };
    check(fd.into())?;
    // SAFETY: the kernel returned a fresh descriptor, owned only here.
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

fn open_directory(path: &Path) -> io::Result<OwnedFd> {
    Ok(fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?
        .into())
}

/// Opens `parent/name` as a directory without following a link, creating it if missing. A link
/// or other file at that name is an error: the harness neither writes through it nor deletes it.
fn subdirectory(parent: &OwnedFd, name: &[u8]) -> io::Result<OwnedFd> {
    let c = c_name(name);
    for _ in 0..4 {
        match openat(parent, name, libc::O_RDONLY | libc::O_DIRECTORY) {
            Ok(fd) => return Ok(fd),
            Err(e) if e.raw_os_error() == Some(libc::ENOENT) => {
                // SAFETY: a valid descriptor and a NUL-terminated name.
                if unsafe { libc::mkdirat(parent.as_raw_fd(), c.as_ptr(), 0o777) } != 0 {
                    let e = io::Error::last_os_error();
                    if e.kind() != io::ErrorKind::AlreadyExists {
                        return Err(e);
                    }
                }
            }
            Err(e) if matches!(e.raw_os_error(), Some(libc::ELOOP | libc::ENOTDIR)) => {
                return Err(io::Error::other(format!(
                    "`{}` is a symbolic link or not a directory; the harness neither follows nor \
                     replaces it, so move it away to publish run artifacts",
                    String::from_utf8_lossy(name)
                )));
            }
            Err(e) => return Err(e),
        }
    }
    Err(io::Error::other("the directory kept changing while the harness opened it"))
}

/// The host's system library directories, which the dynamic loader searches when Landlock denies
/// it its cache (`/etc/ld.so.cache`).
const SYSTEM_DIRS: &[&str] = &[
    "/lib/x86_64-linux-gnu",
    "/usr/lib/x86_64-linux-gnu",
    "/lib64",
    "/usr/lib64",
    "/lib",
    "/usr/lib",
];

/// The interpreter and shared libraries `binary` loads, transitively: absolute names as written,
/// others from the system directories. The driver links every test executable itself, so a
/// library found only through a search path (`DT_RUNPATH`) gets no grant and fails to load.
fn libraries(binary: &Path) -> Result<Vec<PathBuf>, String> {
    let mut found = Vec::new();
    let mut pending = vec![binary.to_path_buf()];
    while let Some(file) = pending.pop() {
        let bytes = fs::read(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        let names = elf_dependencies(&bytes)
            .ok_or_else(|| format!("{}: not a readable 64-bit ELF object", file.display()))?;
        for name in names {
            let path = if name.contains('/') {
                PathBuf::from(&name)
            } else {
                SYSTEM_DIRS
                    .iter()
                    .map(|dir| Path::new(dir).join(&name))
                    .find(|path| path.is_file())
                    .ok_or_else(|| format!("cannot find the library `{name}` the test loads"))?
            };
            if !found.contains(&path) {
                found.push(path.clone());
                pending.push(path);
            }
        }
    }
    Ok(found)
}

/// A little-endian ELF64 object's interpreter (`PT_INTERP`) and needed libraries (`DT_NEEDED`).
fn elf_dependencies(elf: &[u8]) -> Option<Vec<String>> {
    const PT_LOAD: u32 = 1;
    const PT_DYNAMIC: u32 = 2;
    const PT_INTERP: u32 = 3;
    if elf.get(..6)? != b"\x7fELF\x02\x01" {
        return None;
    }
    let at = |offset: u64, len: usize| -> Option<&[u8]> {
        elf.get(usize::try_from(offset).ok()?..)?.get(..len)
    };
    let u16_at = |offset| Some(u16::from_le_bytes(at(offset, 2)?.try_into().ok()?));
    let u32_at = |offset| Some(u32::from_le_bytes(at(offset, 4)?.try_into().ok()?));
    let u64_at = |offset| Some(u64::from_le_bytes(at(offset, 8)?.try_into().ok()?));
    let string = |offset: u64| {
        let rest = elf.get(usize::try_from(offset).ok()?..)?;
        let end = rest.iter().position(|&b| b == 0)?;
        String::from_utf8(rest[..end].to_vec()).ok()
    };
    // (type, file offset, virtual address, file size) of each program header.
    let (table, size, count) = (u64_at(0x20)?, u64::from(u16_at(0x36)?), u16_at(0x38)?);
    let headers = (0..u64::from(count))
        .map(|i| {
            let header = table.checked_add(i.checked_mul(size)?)?;
            Some((
                u32_at(header)?,
                u64_at(header.checked_add(8)?)?,
                u64_at(header.checked_add(16)?)?,
                u64_at(header.checked_add(32)?)?,
            ))
        })
        .collect::<Option<Vec<_>>>()?;
    let mut names = Vec::new();
    if let Some(interp) = headers.iter().find(|h| h.0 == PT_INTERP) {
        names.push(string(interp.1)?);
    }
    let Some(dynamic) = headers.iter().find(|h| h.0 == PT_DYNAMIC) else {
        return Some(names);
    };
    let (mut needed, mut strtab) = (Vec::new(), None);
    for entry in 0..dynamic.3 / 16 {
        let offset = dynamic.1.checked_add(entry * 16)?;
        match (u64_at(offset)?, u64_at(offset.checked_add(8)?)?) {
            (0, _) => break,
            (1, name) => needed.push(name),
            (5, address) => strtab = Some(address),
            _ => {}
        }
    }
    if needed.is_empty() {
        return Some(names);
    }
    // `DT_STRTAB` is a virtual address: map it through the loaded segment that holds it.
    let address = strtab?;
    let segment = headers
        .iter()
        .find(|h| h.0 == PT_LOAD && address >= h.2 && address - h.2 < h.3)?;
    let table = address - segment.2 + segment.1;
    for name in needed {
        names.push(string(table.checked_add(name)?)?);
    }
    Some(names)
}

fn seccomp_available() -> bool {
    let errno = libc::SECCOMP_RET_ERRNO;
    // SAFETY: `PR_GET_SECCOMP` reads no memory; the action query reads one `u32`.
    unsafe {
        libc::prctl(libc::PR_GET_SECCOMP) >= 0
            && libc::syscall(
                libc::SYS_seccomp,
                libc::SECCOMP_GET_ACTION_AVAIL as libc::c_uint,
                0u32,
                &errno,
            ) == 0
    }
}

const AUDIT_ARCH_X86_64: u32 = 0xc000_003e;
const LOAD: u16 = (libc::BPF_LD | libc::BPF_W | libc::BPF_ABS) as u16;
const EQ: u16 = (libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K) as u16;
const SET: u16 = (libc::BPF_JMP | libc::BPF_JSET | libc::BPF_K) as u16;
const RET: u16 = (libc::BPF_RET | libc::BPF_K) as u16;
// Offsets in `struct seccomp_data`: the call number, the architecture, and argument words.
const NR: u32 = 0;
const ARCH: u32 = 4;
const fn low(arg: u32) -> u32 {
    16 + 8 * arg
}
const fn high(arg: u32) -> u32 {
    20 + 8 * arg
}

const fn op(code: u16, k: u32, jt: u8, jf: u8) -> libc::sock_filter {
    libc::sock_filter { code, jt, jf, k }
}

const ALLOW: libc::sock_filter = op(RET, libc::SECCOMP_RET_ALLOW, 0, 0);
const DENY: libc::sock_filter = op(RET, libc::SECCOMP_RET_ERRNO | libc::EPERM as u32, 0, 0);

/// Calls admitted whatever their arguments: memory, threads, futexes, clocks, randomness,
/// polling, signal handling within the process, reads, writes, seeks, and closes on open
/// descriptors, the path operations Landlock confines (opens, directory reads, `mkdir`, rename
/// and unlink of the run's files), `stat`-family and `access` calls, `execve` (Landlock lets it
/// execute only the binary), and exit.
const ALLOWED: &[libc::c_long] = &[
    libc::SYS_read, libc::SYS_write, libc::SYS_readv, libc::SYS_writev, libc::SYS_pread64,
    libc::SYS_pwrite64, libc::SYS_preadv, libc::SYS_pwritev, libc::SYS_preadv2,
    libc::SYS_pwritev2, libc::SYS_lseek, libc::SYS_close, libc::SYS_dup, libc::SYS_dup2,
    libc::SYS_dup3, libc::SYS_pipe2, libc::SYS_fsync, libc::SYS_fdatasync, libc::SYS_ftruncate,
    libc::SYS_openat, libc::SYS_getdents64, libc::SYS_mkdir, libc::SYS_mkdirat, libc::SYS_rename,
    libc::SYS_renameat, libc::SYS_renameat2, libc::SYS_unlink, libc::SYS_unlinkat,
    libc::SYS_stat, libc::SYS_fstat, libc::SYS_lstat, libc::SYS_newfstatat, libc::SYS_statx,
    libc::SYS_access, libc::SYS_faccessat, libc::SYS_faccessat2,
    libc::SYS_brk, libc::SYS_mmap, libc::SYS_munmap, libc::SYS_mprotect, libc::SYS_mremap,
    libc::SYS_madvise, libc::SYS_arch_prctl, libc::SYS_set_tid_address,
    libc::SYS_set_robust_list, libc::SYS_rseq, libc::SYS_futex, libc::SYS_sched_yield,
    libc::SYS_sched_getaffinity, libc::SYS_getpid, libc::SYS_gettid,
    libc::SYS_rt_sigaction, libc::SYS_rt_sigprocmask, libc::SYS_rt_sigreturn,
    libc::SYS_sigaltstack, libc::SYS_tgkill, libc::SYS_restart_syscall,
    libc::SYS_clock_gettime, libc::SYS_clock_getres, libc::SYS_clock_nanosleep,
    libc::SYS_nanosleep, libc::SYS_gettimeofday, libc::SYS_getrandom,
    libc::SYS_poll, libc::SYS_ppoll, libc::SYS_epoll_create1, libc::SYS_epoll_ctl,
    libc::SYS_epoll_wait, libc::SYS_epoll_pwait, libc::SYS_epoll_pwait2, libc::SYS_eventfd2,
    libc::SYS_execve, libc::SYS_exit, libc::SYS_exit_group,
];

/// A block for one call that admits it when argument `arg` (its low word) is one of `values`.
fn one_of(arg: u32, values: &[u32]) -> Vec<libc::sock_filter> {
    let n = values.len();
    let mut block = vec![op(LOAD, low(arg), 0, 0)];
    block.extend(values.iter().enumerate().map(|(i, &v)| op(EQ, v, (n - i) as u8, 0)));
    block.extend([DENY, ALLOW]);
    block
}

fn filter() -> Vec<libc::sock_filter> {
    let mut filter = vec![
        op(LOAD, ARCH, 0, 0),
        op(EQ, AUDIT_ARCH_X86_64, 1, 0),
        op(RET, libc::SECCOMP_RET_KILL_PROCESS, 0, 0),
        op(LOAD, NR, 0, 0),
    ];
    // x32 calls carry the same architecture with bit 30 set, so they match no number here.
    for &call in ALLOWED {
        filter.extend([op(EQ, call as u32, 0, 1), ALLOW]);
    }
    let namespaces = (libc::CLONE_NEWNS
        | libc::CLONE_NEWCGROUP
        | libc::CLONE_NEWUTS
        | libc::CLONE_NEWIPC
        | libc::CLONE_NEWUSER
        | libc::CLONE_NEWPID
        | libc::CLONE_NEWNET) as u32;
    let checked: [(libc::c_long, Vec<libc::sock_filter>); 5] = [
        // Threads only: `clone` without `CLONE_THREAD`, or with a namespace, makes a process.
        (libc::SYS_clone, vec![
            op(LOAD, low(0), 0, 0),
            op(SET, namespaces, 2, 0),
            op(SET, libc::CLONE_THREAD as u32, 0, 1),
            ALLOW,
            DENY,
        ]),
        // `prlimit64` reads limits; it passes only when it sets no new one.
        (libc::SYS_prlimit64, vec![
            op(LOAD, low(2), 0, 0),
            op(EQ, 0, 0, 3),
            op(LOAD, high(2), 0, 0),
            op(EQ, 0, 0, 1),
            ALLOW,
            DENY,
        ]),
        // Terminal queries and descriptor flags; never `TIOCSTI` or a device's own commands.
        (libc::SYS_ioctl, one_of(1, &[libc::TCGETS as u32, libc::FIONBIO as u32, libc::FIOCLEX as u32, libc::FIONCLEX as u32])),
        (libc::SYS_fcntl, one_of(1, &[
            libc::F_DUPFD as u32, libc::F_GETFD as u32, libc::F_SETFD as u32,
            libc::F_GETFL as u32, libc::F_SETFL as u32, libc::F_DUPFD_CLOEXEC as u32,
        ])),
        // Thread names only.
        (libc::SYS_prctl, one_of(0, &[libc::PR_SET_NAME as u32, libc::PR_GET_NAME as u32])),
    ];
    for (call, block) in checked {
        filter.push(op(EQ, call as u32, 0, block.len() as u8));
        filter.extend(block);
    }
    filter.extend([
        op(EQ, libc::SYS_clone3 as u32, 0, 1),
        op(RET, libc::SECCOMP_RET_ERRNO | libc::ENOSYS as u32, 0, 0),
        DENY,
    ]);
    filter
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elf_dependencies_name_the_loader_and_c_library() {
        let names = elf_dependencies(&fs::read(std::env::current_exe().unwrap()).unwrap()).unwrap();
        assert!(names[0].starts_with('/') && names[0].contains("ld-linux"), "{names:?}");
        assert!(names.iter().any(|n| n == "libc.so.6"), "{names:?}");
        assert!(elf_dependencies(b"\x7fELF\x02\x01").is_none());
        assert!(elf_dependencies(b"#!/bin/sh\n").is_none());
    }

    #[test]
    fn levels_name_what_the_kernel_cannot_enforce() {
        assert_eq!(level("7.2", Some(10), true), "full (Linux 7.2, Landlock ABI 10)");
        assert_eq!(level("6.12", Some(6), true), "full (Linux 6.12, Landlock ABI 6)");
        assert_eq!(level("6.8", Some(4), true),
            "partial, not enforced: Landlock scopes for signals and abstract Unix sockets (ABI 6) (Linux 6.8, Landlock ABI 4)");
        assert_eq!(level("5.10", None, true), "partial, not enforced: Landlock (Linux 5.10, no Landlock)");
        assert_eq!(level("7.2", Some(10), false), "partial, not enforced: seccomp (Linux 7.2, Landlock ABI 10)");
        assert_eq!(level("4.19", None, false),
            "partial, not enforced: Landlock, seccomp (Linux 4.19, no Landlock)");
    }

    /// Every jump in the program lands on an instruction, and every path ends in a return.
    #[test]
    fn filter_jumps_stay_inside_the_program() {
        let filter = filter();
        assert!(filter.len() < 4096);
        for (i, insn) in filter.iter().enumerate() {
            if insn.code & 0x07 == libc::BPF_JMP as u16 {
                assert!(i + 1 + usize::from(insn.jt.max(insn.jf)) < filter.len(), "{i}");
            }
        }
        assert_eq!(filter.last().unwrap().code, RET);
    }
}
