// All OS-specific file operations live here; shared wrappers own mode and access policy.
use super::{
    DirEntry, DirEntryBackend, File, FileBackend, Metadata, OpenOptions, checkpoint, split,
};
use crate::io::{self, Write};
use rustix::fd::{AsFd, BorrowedFd, IntoRawFd, OwnedFd};
use rustix::fs::{self, AtFlags, FileType, Mode, OFlags};
use rustix::io::Errno;
use std::ffi::{OsStr, OsString};
use std::mem::MaybeUninit;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub(super) struct Fs {
    base: Option<Arc<OwnedFd>>,
}

pub(super) fn io_error(e: std::io::Error) -> io::Error {
    match e.raw_os_error() {
        Some(code) => io::Error::from_raw_os_error(code),
        None => io::ErrorKind::from_rust(e.kind()).into(),
    }
}

fn os_error(e: Errno) -> io::Error {
    io::Error::from_raw_os_error(e.raw_os_error())
}

fn call<T>(mut f: impl FnMut() -> rustix::io::Result<T>) -> io::Result<T> {
    loop {
        checkpoint()?;
        match f() {
            Err(Errno::INTR) => continue,
            result => return result.map_err(os_error),
        }
    }
}

fn confined_open(mut open: impl FnMut() -> rustix::io::Result<OwnedFd>) -> io::Result<OwnedFd> {
    for retry in 0..=4 {
        match call(&mut open) {
            Err(e) if e.raw_os_error() == Some(Errno::AGAIN.raw_os_error()) && retry < 4 => {
                continue;
            }
            Err(e) if e.raw_os_error() == Some(Errno::XDEV.raw_os_error()) => {
                return Err(io::ErrorKind::PermissionDenied.into());
            }
            Err(e) if e.raw_os_error() == Some(Errno::NOSYS.raw_os_error()) => {
                return Err(io::ErrorKind::Unsupported.into());
            }
            result => return result,
        }
    }
    unreachable!()
}

fn resolve_from(
    base: BorrowedFd<'_>,
    confined: bool,
    p: &Path,
    flags: OFlags,
    mode: Mode,
) -> io::Result<OwnedFd> {
    let flags = flags | OFlags::CLOEXEC;
    if confined {
        confined_open(|| {
            fs::openat2(
                base,
                p,
                flags,
                mode,
                fs::ResolveFlags::BENEATH | fs::ResolveFlags::NO_MAGICLINKS,
            )
        })
    } else {
        call(|| fs::openat(base, p, flags, mode))
    }
}

impl Fs {
    pub(super) fn is_confined(&self) -> bool {
        self.base.is_some()
    }

    fn resolve(&self, p: &Path, flags: OFlags, mode: Mode) -> io::Result<OwnedFd> {
        let base = self.base.as_deref().map_or(fs::CWD, |fd| fd.as_fd());
        resolve_from(base, self.is_confined(), p, flags, mode)
    }

    pub(super) fn dir(&self, p: &Path) -> io::Result<Self> {
        Ok(Self {
            base: Some(Arc::new(self.resolve(
                p,
                OFlags::RDONLY | OFlags::DIRECTORY,
                Mode::empty(),
            )?)),
        })
    }

    pub(super) fn open(&self, p: &Path, opts: &OpenOptions) -> io::Result<std::fs::File> {
        let mut flags = match (opts.read, opts.write || opts.append) {
            (true, true) => OFlags::RDWR,
            (true, false) => OFlags::RDONLY,
            (false, true) => OFlags::WRONLY,
            (false, false) => return Err(io::ErrorKind::InvalidInput.into()),
        };
        if opts.append {
            flags |= OFlags::APPEND;
        }
        if opts.create_new {
            flags |= OFlags::CREATE | OFlags::EXCL;
        } else {
            if opts.create {
                flags |= OFlags::CREATE;
            }
            if opts.truncate {
                flags |= OFlags::TRUNC;
            }
        }
        let mode = if flags.contains(OFlags::CREATE) {
            Mode::from_raw_mode(0o666)
        } else {
            Mode::empty()
        };
        Ok(self.resolve(p, flags, mode)?.into())
    }

    pub(super) fn metadata(&self, p: &Path) -> io::Result<Metadata> {
        let fd = self.resolve(p, OFlags::PATH, Mode::empty())?;
        metadata(&fd)
    }

    fn parent(&self, p: &Path) -> io::Result<(OwnedFd, OsString)> {
        super::normal_name(p)?;
        let (parent, name) = split(p)?;
        Ok((
            self.resolve(&parent, OFlags::PATH | OFlags::DIRECTORY, Mode::empty())?,
            name,
        ))
    }

    pub(super) fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        let (from_dir, from) = self.parent(from)?;
        let (to_dir, to) = self.parent(to)?;
        call(|| fs::renameat(&from_dir, &from, &to_dir, &to))
    }

    pub(super) fn remove(&self, p: &Path, directory: bool) -> io::Result<()> {
        let (parent, name) = self.parent(p)?;
        let flags = if directory {
            AtFlags::REMOVEDIR
        } else {
            AtFlags::empty()
        };
        call(|| fs::unlinkat(&parent, &name, flags))
    }

    pub(super) fn sync_dir(&self, p: &Path) -> io::Result<()> {
        let fd = self.resolve(p, OFlags::RDONLY | OFlags::DIRECTORY, Mode::empty())?;
        call(|| fs::fsync(&fd))
    }

    pub(super) fn write_synced(&self, p: &Path, data: &[u8]) -> io::Result<()> {
        let mut path = p.to_owned();
        // Resolve final symlinks explicitly so we can sync the target's pinned parent,
        // including when a dangling symlink creates a file in a different directory.
        for links in 0..=40 {
            let (parent_path, name) = split(&path)?;
            let parent = self.resolve(
                &parent_path,
                OFlags::RDONLY | OFlags::DIRECTORY,
                Mode::empty(),
            )?;
            let flags = OFlags::WRONLY
                | OFlags::CREATE
                | OFlags::TRUNC
                | OFlags::NOFOLLOW
                | OFlags::CLOEXEC;
            let opened = resolve_from(
                parent.as_fd(),
                self.is_confined(),
                Path::new(&name),
                flags,
                Mode::from_raw_mode(0o666),
            );
            match opened {
                Ok(fd) => {
                    let mut file = File {
                        backend: FileBackend::Real(fd.into()),
                        read: false,
                        write: true,
                    };
                    file.write_all(data)?;
                    file.sync()?;
                    return call(|| fs::fsync(&parent));
                }
                Err(e) if e.raw_os_error() == Some(Errno::LOOP.raw_os_error()) && links < 40 => {
                    let target = read_link(&parent, &name)?;
                    super::check_path(&target, self.is_confined())?;
                    path = parent_path.join(target);
                }
                Err(e) => return Err(e),
            }
        }
        Err(os_error(Errno::LOOP))
    }

    pub(super) fn create_dir_all(&self, p: &Path) -> io::Result<()> {
        let mut prefix = PathBuf::new();
        for component in p.components() {
            prefix.push(component);
            match self.resolve(&prefix, OFlags::PATH | OFlags::DIRECTORY, Mode::empty()) {
                Ok(_) => continue,
                Err(e)
                    if e.kind() == io::ErrorKind::NotFound
                        && matches!(component, Component::Normal(_)) => {}
                Err(e) if e.kind() == io::ErrorKind::NotADirectory => {
                    // Let the target mkdir distinguish an existing file from a file parent.
                    let (parent_path, name) = split(p)?;
                    let parent = self.resolve(
                        &parent_path,
                        OFlags::PATH | OFlags::DIRECTORY,
                        Mode::empty(),
                    )?;
                    super::normal_name(p)?;
                    return call(|| fs::mkdirat(&parent, &name, Mode::from_raw_mode(0o777)));
                }
                Err(e) => return Err(e),
            }
            let (parent, name) = self.parent(&prefix)?;
            match call(|| fs::mkdirat(&parent, &name, Mode::from_raw_mode(0o777))) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                    match self.resolve(&prefix, OFlags::PATH | OFlags::DIRECTORY, Mode::empty()) {
                        Ok(_) => {}
                        Err(check)
                            if matches!(
                                check.kind(),
                                io::ErrorKind::Cancelled
                                    | io::ErrorKind::PermissionDenied
                                    | io::ErrorKind::Unsupported
                                    | io::ErrorKind::WouldBlock
                            ) =>
                        {
                            return Err(check);
                        }
                        Err(_) => return Err(e),
                    }
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    pub(super) fn read_dir(&self, p: &Path) -> io::Result<Vec<DirEntry>> {
        let fd = Arc::new(self.resolve(p, OFlags::RDONLY | OFlags::DIRECTORY, Mode::empty())?);
        let names = names(&fd)?;
        Ok(names
            .into_iter()
            .map(|name| DirEntry {
                backend: DirEntryBackend::Real(Entry {
                    directory: fd.clone(),
                }),
                path: p.join(&name),
                name,
            })
            .collect())
    }

    pub(super) fn remove_dir_all(&self, p: &Path) -> io::Result<()> {
        // A spelling ending in `.`/`..` can name the base; empty it without unlinking it.
        if super::normal_name(p).is_err() && !p.as_os_str().is_empty() {
            let fd = self.resolve(p, OFlags::RDONLY | OFlags::DIRECTORY, Mode::empty())?;
            empty_dir(&fd)?;
            return Err(io::ErrorKind::InvalidInput.into());
        }
        let (parent, name) = self.parent(p)?;
        remove_entry(&parent, &name)
    }
}

fn metadata(fd: impl AsFd) -> io::Result<Metadata> {
    Ok(from_stat(call(|| fs::fstat(fd.as_fd()))?))
}

pub(super) fn file_metadata(file: &std::fs::File) -> io::Result<Metadata> {
    metadata(file)
}

pub(super) fn sync_all(file: &std::fs::File) -> io::Result<()> {
    call(|| fs::fsync(file))
}

pub(super) fn sync_data(file: &std::fs::File) -> io::Result<()> {
    call(|| fs::fdatasync(file))
}

pub(super) fn set_len(file: &std::fs::File, n: u64) -> io::Result<()> {
    call(|| fs::ftruncate(file, n))
}

fn from_stat(stat: fs::Stat) -> Metadata {
    Metadata::new(
        match FileType::from_raw_mode(stat.st_mode) {
            FileType::RegularFile => super::Kind::File,
            FileType::Directory => super::Kind::Dir,
            FileType::Symlink => super::Kind::Symlink,
            _ => super::Kind::Other,
        },
        stat.st_size as u64,
    )
}

#[derive(Debug)]
pub(super) struct Entry {
    directory: Arc<OwnedFd>,
}
impl Entry {
    pub(super) fn metadata(&self, name: &OsStr) -> io::Result<Metadata> {
        Ok(from_stat(call(|| {
            fs::statat(&*self.directory, name, AtFlags::SYMLINK_NOFOLLOW)
        })?))
    }
}

fn read_link(parent: &OwnedFd, name: &OsStr) -> io::Result<PathBuf> {
    let mut buf = vec![0; 256];
    loop {
        let len = call(|| fs::readlinkat_raw(parent, name, buf.as_mut_slice()))?;
        if len < buf.len() {
            buf.truncate(len);
            return Ok(OsString::from_vec(buf).into());
        }
        buf.resize(buf.len() * 2, 0);
    }
}

fn names(fd: &OwnedFd) -> io::Result<Vec<OsString>> {
    // Callers supply a freshly opened directory, so no fd duplication or rewind is needed.
    let mut buf = vec![MaybeUninit::uninit(); 8192];
    let mut names = Vec::new();
    loop {
        let mut dir = fs::RawDir::new(fd, &mut buf);
        loop {
            let entry = call(|| {
                dir.next().transpose().map(|entry| {
                    entry.map(|e| OsString::from_vec(e.file_name().to_bytes().to_vec()))
                })
            });
            match entry {
                Ok(Some(name)) if name != "." && name != ".." => names.push(name),
                Ok(Some(_)) => {}
                Ok(None) => {
                    names.sort();
                    return Ok(names);
                }
                Err(e) if e.raw_os_error() == Some(Errno::INVAL.raw_os_error()) => break,
                Err(e) => return Err(e),
            }
        }
        buf.resize(buf.len() * 2, MaybeUninit::uninit());
    }
}

fn empty_dir(fd: &OwnedFd) -> io::Result<()> {
    for name in names(fd)? {
        remove_entry(fd, &name)?;
    }
    Ok(())
}

fn remove_entry(parent: &OwnedFd, name: &OsStr) -> io::Result<()> {
    let bytes = name.as_bytes();
    let bare = OsStr::from_bytes(super::trim_final_slashes(bytes));
    let stat = call(|| fs::statat(parent, bare, AtFlags::SYMLINK_NOFOLLOW))?;
    if bytes.ends_with(b"/") && FileType::from_raw_mode(stat.st_mode) != FileType::Directory {
        return Err(io::ErrorKind::NotADirectory.into());
    }
    let name = bare;
    if FileType::from_raw_mode(stat.st_mode) == FileType::Directory {
        // Do not follow a swapped-in symlink during recursive removal.
        let child = call(|| {
            fs::openat(
                parent,
                name,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
        })?;
        empty_dir(&child)?;
        call(|| fs::unlinkat(parent, name, AtFlags::REMOVEDIR))
    } else {
        call(|| fs::unlinkat(parent, name, AtFlags::empty()))
    }
}

pub(super) fn close(file: std::fs::File) -> io::Result<()> {
    unsafe extern "C" {
        fn close(fd: std::ffi::c_int) -> std::ffi::c_int;
    }
    // Linux releases the fd even on error. Transfer ownership once and never retry close.
    if unsafe { close(file.into_raw_fd()) } == 0 {
        Ok(())
    } else {
        Err(io_error(std::io::Error::last_os_error()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_stops_later_calls_and_interrupted_truncate_retries() {
        let path = std::env::temp_dir().join(format!("tide-truncate-{}", std::process::id()));
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        // Keep the test file anonymous so even a failed assertion leaves no artifact.
        std::fs::remove_file(path).unwrap();
        std::io::Write::write_all(&mut &file, b"unchanged").unwrap();
        for interrupted in [false, true] {
            let mut attempts = 0;
            crate::task::Tasks::real()
                .scope(|scope| -> Result<(), crate::task::TaskError> {
                    let result = call(|| {
                        attempts += 1;
                        if attempts == 1 {
                            scope.cancel();
                            if interrupted {
                                return Err(Errno::INTR);
                            }
                            return Ok(());
                        }
                        fs::ftruncate(&file, 0)
                    });
                    if interrupted {
                        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Cancelled);
                    } else {
                        result.unwrap(); // An already completed call keeps its own result.
                        assert_eq!(
                            call(|| fs::ftruncate(&file, 0)).unwrap_err().kind(),
                            io::ErrorKind::Cancelled
                        );
                    }
                    Ok(())
                })
                .unwrap();
            assert_eq!(attempts, 1);
            assert_eq!(file.metadata().unwrap().len(), 9);
        }

        let mut attempts = 0;
        call(|| {
            attempts += 1;
            if attempts == 1 {
                Err(Errno::INTR)
            } else {
                fs::ftruncate(&file, 0)
            }
        })
        .unwrap();
        assert_eq!(attempts, 2);
        assert_eq!(file.metadata().unwrap().len(), 0);
    }

    #[test]
    fn resolution_fails_closed_and_only_resolution_maps_cross_device() {
        let e = confined_open(|| Err(Errno::NOSYS)).unwrap_err();
        assert_eq!(e.kind(), io::ErrorKind::Unsupported);
        assert_eq!(
            confined_open(|| Err(Errno::XDEV)).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
        assert_eq!(os_error(Errno::XDEV).kind(), io::ErrorKind::CrossesDevices);
        let mut attempts = 0;
        let e = confined_open(|| {
            attempts += 1;
            Err(Errno::AGAIN)
        })
        .unwrap_err();
        assert_eq!(attempts, 5);
        assert_eq!(e.kind(), io::ErrorKind::WouldBlock);
    }
}

// Preserve TASK-3 checks between the syscalls of provided Read methods.
pub(super) struct Reader<'a>(pub(super) &'a std::fs::File);
impl crate::io::Read for Reader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        checkpoint()?;
        std::io::Read::read(&mut self.0, buf).map_err(io_error)
    }
}
